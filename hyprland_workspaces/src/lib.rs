use hyprland::data::Workspace as HyprWorkspace;
use hyprland::prelude::*;
use hyprland::shared::HyprError;
use std::collections::HashMap;
pub fn workspaces() -> anyhow::Result<Vec<Workspace>> {
    let workspaces = hyprland::data::Workspaces::get()?.to_vec();
    let mut workspace_icons = (1..10)
        .into_iter()
        .map(|id| (id, WorkspaceState::Unused))
        .collect::<HashMap<_, _>>();
    for workspace in workspaces {
        if workspace.windows > 0 {
            workspace_icons.insert(workspace.id, WorkspaceState::Used);
        }
    }
    workspace_icons.insert(HyprWorkspace::get_active()?.id, WorkspaceState::Active);
    let mut workspace_icons = workspace_icons
        .into_iter()
        .map(|(id, state)| Workspace { id, state })
        .collect::<Vec<_>>();
    workspace_icons.sort_by_key(|w| w.id);
    Ok(workspace_icons)
}
pub fn change_workspace(id: i32) -> anyhow::Result<(), HyprError> {
    hyprland::dispatch::Dispatch::call(hyprland::dispatch::DispatchType::Workspace(
        hyprland::dispatch::WorkspaceIdentifierWithSpecial::Id(id),
    ))
}

#[derive(Debug, Clone, Copy)]
pub enum WorkspaceState {
    Active,
    Used,
    Unused,
}
pub struct Workspace {
    pub id: i32,
    pub state: WorkspaceState,
}
