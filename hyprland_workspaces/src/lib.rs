use hyprland::data::Workspace as HyprWorkspace;
use hyprland::prelude::*;
use hyprland::shared::HyprError;
use std::collections::HashMap;

use embargo_workspace::WorkspaceState;
pub fn workspaces() -> anyhow::Result<Vec<HyprlandWorkspace>> {
    let workspaces = hyprland::data::Workspaces::get()?.to_vec();
    let mut workspace_icons = (1..10)
        .map(|id| (id as u32, WorkspaceState::Unused))
        .collect::<HashMap<_, _>>();
    for workspace in workspaces {
        if workspace.windows > 0 {
            workspace_icons.insert(workspace.id as u32, WorkspaceState::Used);
        }
    }
    workspace_icons.insert(
        HyprWorkspace::get_active()?.id as u32,
        WorkspaceState::Active,
    );
    let mut workspace_icons = workspace_icons
        .into_iter()
        .map(|(id, state)| HyprlandWorkspace {
            position: id,
            state,
        })
        .collect::<Vec<_>>();
    workspace_icons.sort_by_key(|w| w.position);
    Ok(workspace_icons)
}

pub fn change_workspace(id: u32) -> Result<(), HyprError> {
    hyprland::dispatch::Dispatch::call(hyprland::dispatch::DispatchType::Workspace(
        hyprland::dispatch::WorkspaceIdentifierWithSpecial::Id(id as i32),
    ))
}

pub struct HyprlandWorkspace {
    pub position: u32,
    pub state: WorkspaceState,
}

impl embargo_workspace::Workspace for HyprlandWorkspace {
    fn state(&self) -> WorkspaceState {
        self.state
    }
    fn position(&self) -> u32 {
        self.position
    }
}
