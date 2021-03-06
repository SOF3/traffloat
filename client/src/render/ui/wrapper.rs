use std::cell::RefCell;
use std::rc::Rc;

use yew::prelude::*;

use super::edge_preview;
use super::node_preview;

/// Wrapper for UI elements.
pub struct Wrapper {
    props: Props,
    link: ComponentLink<Self>,
    node_preview_info: Option<node_preview::Props>,
    edge_preview_info: Option<edge_preview::Props>,
}

impl Component for Wrapper {
    type Message = Update;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        props.updater_ref.set(link.callback(|update| update));

        Self {
            props,
            link,
            node_preview_info: None,
            edge_preview_info: None,
        }
    }

    fn update(&mut self, msg: Update) -> ShouldRender {
        match msg {
            Update::SetNodePreview(props) => {
                match (&self.node_preview_info, &props) {
                    (None, None) => return false,
                    (Some(old), Some(new)) if old.entity == new.entity => return false,
                    _ => (),
                }
                self.node_preview_info = props;
                true
            }
            Update::SetEdgePreview(props) => {
                match (&self.edge_preview_info, &props) {
                    (None, None) => return false,
                    (Some(old), Some(new)) if old.entity == new.entity => return false,
                    _ => (),
                }
                self.edge_preview_info = props;
                true
            }
        }
    }

    fn change(&mut self, props: Props) -> ShouldRender {
        props.updater_ref.set(self.link.callback(|update| update));
        self.props = props;
        false // we just modified the setter, but there is nothing to render yet
    }

    fn view(&self) -> Html {
        html! {
            <div style="
                z-index: 3;
                position: absolute;
                width: 100vw; height: 100vh;
                pointer-events: none;
                x: 0; y: 0;
            ">
                { for self.node_preview_info.as_ref().map(|props| html! {
                    <node_preview::Comp with props.clone() />
                }) }
                { for self.edge_preview_info.as_ref().map(|props| html! {
                    <edge_preview::Comp with props.clone() />
                }) }
            </div>
        }
    }
}

/// Events for [`Wrapper`].
pub enum Update {
    /// Sets the node preview info to display.
    SetNodePreview(Option<node_preview::Props>),
    /// Sets the edge preview info to display.
    SetEdgePreview(Option<edge_preview::Props>),
}

/// Yew properties for [`Wrapper`].
#[derive(Clone, Properties)]
pub struct Props {
    /// An interiorly-mutable reference to update the yew callback for UI messages [`Update`].
    pub updater_ref: UpdaterRef,
}

/// An interiorly-mutable reference to update the yew callback for UI messages [`Update`].
#[derive(Clone, Default)]
pub struct UpdaterRef {
    cell: Rc<RefCell<Option<Callback<Update>>>>,
}

impl UpdaterRef {
    /// Updates the callback.
    pub fn set(&self, callback: Callback<Update>) {
        let _ = self.cell.replace(Some(callback));
    }

    /// Invokes the callback if it exists.
    pub fn call(&self, update: Update) {
        if let Some(callback) = &*self.cell.borrow() {
            callback.emit(update);
        }
    }
}
