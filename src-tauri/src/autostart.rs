use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Enable,
    Disable,
}

impl Action {
    fn for_config(enabled: bool) -> Action {
        if enabled {
            Action::Enable
        } else {
            Action::Disable
        }
    }

    fn name(self) -> &'static str {
        match self {
            Action::Enable => "enable",
            Action::Disable => "disable",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Applied(Action),
    Failed { action: Action, error: String },
}

fn outcome(action: Action, result: Result<(), String>) -> Outcome {
    match result {
        Ok(()) => Outcome::Applied(action),
        Err(error) => Outcome::Failed { action, error },
    }
}

pub fn apply(app: &AppHandle, enabled: bool) {
    let action = Action::for_config(enabled);
    let manager = app.autolaunch();
    let result = match action {
        Action::Enable => manager.enable(),
        Action::Disable => manager.disable(),
    };

    match outcome(action, result.map_err(|error| error.to_string())) {
        Outcome::Applied(action) => {
            tracing::debug!(action = action.name(), "autostart applied");
        }
        Outcome::Failed { action, error } => {
            tracing::warn!(
                action = action.name(),
                %error,
                "failed to apply autostart"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Action, Outcome, outcome};

    #[test]
    fn config_true_maps_to_enable() {
        assert_eq!(Action::for_config(true), Action::Enable);
    }

    #[test]
    fn config_false_maps_to_disable() {
        assert_eq!(Action::for_config(false), Action::Disable);
    }

    #[test]
    fn success_reports_the_applied_action() {
        assert_eq!(
            outcome(Action::Enable, Ok(())),
            Outcome::Applied(Action::Enable)
        );
        assert_eq!(
            outcome(Action::Disable, Ok(())),
            Outcome::Applied(Action::Disable)
        );
    }

    #[test]
    fn failure_keeps_the_action_and_the_plugin_error() {
        assert_eq!(
            outcome(
                Action::Disable,
                Err("desktop file is read-only".to_string())
            ),
            Outcome::Failed {
                action: Action::Disable,
                error: "desktop file is read-only".to_string(),
            }
        );
    }
}
