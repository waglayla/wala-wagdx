use std::ops::{Deref, DerefMut};
use crate::imports::*;
use super::*;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

pub const FOCUS_NONE: u32 = 0;

#[derive(Clone, Default)]
pub struct FocusContext {
  pub focus: u32,
}

pub trait HasFocusContext {
  fn focus_context(&self) -> &FocusContext;
  fn focus_context_mut(&mut self) -> &mut FocusContext;
}

pub trait FocusTrait: HasFocusContext {
  fn get_focus(&self) -> u32 {
    self.focus_context().focus
  }
  
  fn assign_focus(&mut self, new_focus: impl ToPrimitive) {
    self.focus_context_mut().focus = new_focus.to_u32().unwrap_or(FOCUS_NONE);
  }

  fn next_focus(&mut self, ui: &mut egui::Ui, focus: impl ToPrimitive, elem: egui::Response) {
    if let Some(focus_val) = focus.to_u32() {
      if self.get_focus() == focus_val {
        set_focus(ui, elem);
        self.focus_context_mut().focus = FOCUS_NONE;
      }
    }
  }
}

impl<T: HasFocusContext> FocusTrait for T {}

#[macro_export]
macro_rules! impl_has_focus_context {
  ($type:ty) => {
    impl HasFocusContext for $type {
      fn focus_context(&self) -> &FocusContext {
        &self.focus_context
      }
      
      fn focus_context_mut(&mut self) -> &mut FocusContext {
        &mut self.focus_context
      }
    }
  };
}

pub(crate) use impl_has_focus_context;