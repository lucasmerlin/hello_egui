use crate::history::{DefaultHistory, History};
use crate::route_kind::RouteKind;
use crate::router_builder::{ErrorUi, RouterBuilder};
use crate::transition::{ActiveTransition, ActiveTransitionResult};
use crate::{
    CurrentTransition, Request, RouteState, RouterError, RouterResult, TransitionConfig, ID,
};
use egui::Ui;
use matchit::MatchError;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::atomic::Ordering;

/// A router instance
pub struct EguiRouter<State, History = DefaultHistory> {
    router: matchit::Router<RouteKind<State>>,
    history: Vec<RouteState<State>>,

    history_kind: History,

    forward_transition: TransitionConfig,
    backward_transition: TransitionConfig,
    replace_transition: TransitionConfig,

    current_transition: Option<CurrentTransition<State>>,
    default_duration: Option<f32>,

    error_ui: ErrorUi<State>,
}

impl<State: 'static, H: History + Default> EguiRouter<State, H> {
    /// Create a new [`RouterBuilder`]
    pub fn builder() -> RouterBuilder<State, H> {
        RouterBuilder::new()
    }

    pub(crate) fn from_builder(builder: RouterBuilder<State, H>, state: &mut State) -> Self {
        let mut router = Self {
            router: builder.router,
            history: Vec::new(),
            history_kind: builder.history_kind.unwrap_or_default(),
            current_transition: None,
            forward_transition: builder.forward_transition,
            backward_transition: builder.backward_transition,
            replace_transition: builder.replace_transition,
            default_duration: builder.default_duration,
            error_ui: builder.error_ui,
        };

        if let Some((r, state_index)) = router
            .history_kind
            .active_route()
            .or(builder.default_route.map(|d| (d, None)))
        {
            router
                .navigate_impl(
                    state,
                    &r,
                    TransitionConfig::none(),
                    state_index.unwrap_or(0),
                )
                .unwrap();
        }

        router
    }

    /// Get the active route
    pub fn active_route(&self) -> Option<&str> {
        self.history.last().map(|r| r.path.as_str())
    }

    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    pub fn history(&self) -> impl Iterator<Item = &str> {
        self.history.iter().map(|s| s.path.as_str())
    }

    fn parse_path(path: &str) -> (&str, BTreeMap<Cow<str>, Cow<str>>) {
        path.split_once('?')
            .map(|(path, q)| (path, form_urlencoded::parse(q.as_bytes()).collect()))
            .unwrap_or((path, BTreeMap::new()))
    }

    fn navigate_impl(
        &mut self,
        state: &mut State,
        path: &str,
        transition_config: TransitionConfig,
        new_state: u32,
    ) -> RouterResult {
        let (path, query) = Self::parse_path(path);

        let mut redirect = None;
        let result = self.router.at_mut(path);

        let result = match result {
            Ok(match_) => {
                match match_.value {
                    RouteKind::Route(handler) => {
                        let route = handler(Request {
                            state,
                            params: match_.params,
                            query,
                        });
                        self.history.push(RouteState {
                            path: path.to_string(),
                            route,
                            id: ID.fetch_add(1, Ordering::SeqCst),
                            state: new_state,
                        });

                        self.current_transition = Some(CurrentTransition {
                            active_transition: ActiveTransition::forward(transition_config.clone())
                                .with_default_duration(self.default_duration),
                            leaving_route: None,
                        });
                    }
                    RouteKind::Redirect(r) => {
                        redirect = Some(r.clone());
                    }
                }
                Ok(())
            }
            Err(e) => match e {
                MatchError::NotFound => Err(RouterError::NotFound),
            },
        };

        if let Some(redirect) = redirect {
            self.history_kind.replace(&redirect, new_state)?;
            self.navigate_impl(state, &redirect, transition_config, new_state)?;
        }

        result
    }

    /// Navigate with a custom transition
    pub fn navigate_transition(
        &mut self,
        state: &mut State,
        path: impl Into<String>,
        transition_config: TransitionConfig,
    ) -> RouterResult {
        let path = path.into();
        let current_state = self.history.last().map_or(0, |r| r.state);
        let new_state = current_state + 1;
        self.history_kind.push(&path, new_state)?;
        self.navigate_impl(state, &path, transition_config, new_state)?;
        Ok(())
    }

    /// Navigate with the default transition
    pub fn navigate(&mut self, state: &mut State, route: impl Into<String>) -> RouterResult {
        self.navigate_transition(state, route, self.forward_transition.clone())
    }

    fn back_impl(&mut self, transition_config: TransitionConfig) {
        if self.history.len() > 1 {
            let leaving_route = self.history.pop();
            self.current_transition = Some(CurrentTransition {
                active_transition: ActiveTransition::backward(transition_config)
                    .with_default_duration(self.default_duration),
                leaving_route,
            });
        }
    }

    /// Go back with a custom transition
    pub fn back_transition(&mut self, transition_config: TransitionConfig) -> RouterResult {
        self.history_kind.back()?;
        self.back_impl(transition_config);
        Ok(())
    }

    /// Go back with the default transition
    pub fn back(&mut self) -> RouterResult {
        self.back_transition(self.backward_transition.clone())
    }

    /// Replace the current route with a custom transition
    pub fn replace_transition(
        &mut self,
        state: &mut State,
        path: impl Into<String>,
        transition_config: TransitionConfig,
    ) -> RouterResult {
        let mut redirect = None;

        let path = path.into();
        let result = self.router.at_mut(&path);

        let current_state = self.history.last().map_or(0, |r| r.state);
        let new_state = current_state;

        let (path, query) = Self::parse_path(&path);

        let result = match result {
            Ok(match_) => match match_.value {
                RouteKind::Route(handler) => {
                    self.history_kind.replace(path, new_state)?;
                    let leaving_route = self.history.pop();
                    let route = handler(Request {
                        state,
                        params: match_.params,
                        query,
                    });
                    self.history.push(RouteState {
                        path: path.to_string(),
                        route,
                        id: ID.fetch_add(1, Ordering::SeqCst),
                        state: new_state,
                    });

                    self.current_transition = Some(CurrentTransition {
                        active_transition: ActiveTransition::forward(transition_config.clone())
                            .with_default_duration(self.default_duration),
                        leaving_route,
                    });

                    Ok(())
                }
                RouteKind::Redirect(r) => {
                    redirect = Some(r.clone());
                    Ok(())
                }
            },
            Err(MatchError::NotFound) => Err(RouterError::NotFound),
        };

        if let Some(redirect) = redirect {
            self.history_kind.replace(&redirect, new_state)?;
            self.replace_transition(state, redirect, transition_config)?;
        }

        result
    }

    /// Replace the current route with the default transition
    pub fn replace(&mut self, state: &mut State, path: impl Into<String>) -> RouterResult {
        self.replace_transition(state, path, self.replace_transition.clone())
    }

    /// Render the router
    pub fn ui(&mut self, ui: &mut Ui, state: &mut State) {
        for e in self.history_kind.update(ui.ctx()) {
            let state_index = e.state.unwrap_or(0);
            let path = e.location;

            if let Some(route_state) = self
                .history
                .iter()
                .find(|r| r.path == path && r.state == state_index)
                .map(|r| r.state)
            {
                let active_state = self.history.last().map_or(0, |r| r.state);

                // Retain all routes with a state less than or equal to the new state and the active state so that we can animate them out
                self.history
                    .retain(|r| r.state <= route_state || r.state == active_state);

                if route_state < active_state {
                    self.back_impl(self.backward_transition.clone());
                }
            } else {
                self.navigate_impl(state, &path, self.forward_transition.clone(), state_index)
                    .ok();
            }
        }

        if let Some((last, previous)) = self.history.split_last_mut() {
            let result = if let Some(transition) = &mut self.current_transition {
                let leaving_route_state = transition.leaving_route.as_mut().or(previous.last_mut());
                Some(transition.active_transition.show(
                    ui,
                    state,
                    (last.id, |ui, state| match &mut last.route {
                        Ok(route) => {
                            route.ui(ui, state);
                        }
                        Err(err) => {
                            (self.error_ui)(ui, state, err);
                        }
                    }),
                    leaving_route_state.map(|r| {
                        (r.id, |ui: &mut Ui, state: &mut _| match &mut r.route {
                            Ok(route) => {
                                route.ui(ui, state);
                            }
                            Err(err) => {
                                (self.error_ui)(ui, state, err);
                            }
                        })
                    }),
                ))
            } else {
                ActiveTransition::show_default(ui, last.id, |ui| match &mut last.route {
                    Ok(route) => {
                        route.ui(ui, state);
                    }
                    Err(err) => {
                        (self.error_ui)(ui, state, err);
                    }
                });
                None
            };

            match result {
                Some(ActiveTransitionResult::Done) => {
                    self.current_transition = None;
                }
                Some(ActiveTransitionResult::Continue) | None => {}
            }
        }
    }
}
