use std::time::Duration;
use std::time::Instant;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;

use crate::key_hint::KeyBinding;

const ESC_INTERRUPT_WINDOW: Duration = Duration::from_millis(200);

#[derive(Debug, Default)]
pub(crate) struct EscInterruptArmer {
    armed_until: Option<Instant>,
}

impl EscInterruptArmer {
    pub(crate) fn is_escape_event(key_event: KeyEvent) -> bool {
        key_event.code == KeyCode::Esc
    }

    pub(crate) fn is_interrupt_trigger(key_event: KeyEvent) -> bool {
        Self::is_escape_event(key_event) && key_event.kind == KeyEventKind::Press
    }

    pub(crate) fn is_configured_escape_interrupt(
        interrupt_turn_keys: &[KeyBinding],
        key_event: KeyEvent,
    ) -> bool {
        Self::is_escape_event(key_event)
            && interrupt_turn_keys.iter().any(|binding| {
                let (key, modifiers) = binding.parts();
                key == KeyCode::Esc && modifiers == key_event.modifiers
            })
    }

    pub(crate) fn confirm_or_arm(&mut self) -> bool {
        self.confirm_or_arm_at(Instant::now())
    }

    fn confirm_or_arm_at(&mut self, now: Instant) -> bool {
        let confirmed = self.armed_until.is_some_and(|until| now <= until);
        if confirmed {
            self.armed_until = None;
        } else {
            self.armed_until = now.checked_add(ESC_INTERRUPT_WINDOW);
        }
        confirmed
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::KeyModifiers;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn interrupt_trigger_only_matches_esc_press() {
        assert!(EscInterruptArmer::is_interrupt_trigger(KeyEvent::new(
            KeyCode::Esc,
            KeyModifiers::NONE
        )));
        assert!(!EscInterruptArmer::is_interrupt_trigger(
            KeyEvent::new_with_kind(KeyCode::Esc, KeyModifiers::NONE, KeyEventKind::Repeat)
        ));
        assert!(!EscInterruptArmer::is_interrupt_trigger(
            KeyEvent::new_with_kind(KeyCode::Esc, KeyModifiers::NONE, KeyEventKind::Release)
        ));
        assert!(!EscInterruptArmer::is_interrupt_trigger(KeyEvent::new(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL
        )));
    }

    #[test]
    fn configured_escape_interrupt_matches_escape_independent_of_kind() {
        let bindings = [crate::key_hint::plain(KeyCode::Esc)];

        assert!(EscInterruptArmer::is_configured_escape_interrupt(
            &bindings,
            KeyEvent::new_with_kind(KeyCode::Esc, KeyModifiers::NONE, KeyEventKind::Press)
        ));
        assert!(EscInterruptArmer::is_configured_escape_interrupt(
            &bindings,
            KeyEvent::new_with_kind(KeyCode::Esc, KeyModifiers::NONE, KeyEventKind::Repeat)
        ));
        assert!(EscInterruptArmer::is_configured_escape_interrupt(
            &bindings,
            KeyEvent::new_with_kind(KeyCode::Esc, KeyModifiers::NONE, KeyEventKind::Release)
        ));
    }

    #[test]
    fn configured_escape_interrupt_ignores_remapped_keys() {
        let bindings = [crate::key_hint::plain(KeyCode::F(12))];

        assert!(!EscInterruptArmer::is_configured_escape_interrupt(
            &bindings,
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)
        ));
    }

    #[test]
    fn second_press_inside_window_confirms_interrupt() {
        let mut armer = EscInterruptArmer::default();
        let now = Instant::now();

        assert_eq!(armer.confirm_or_arm_at(now), false);
        assert_eq!(
            armer.confirm_or_arm_at(now + Duration::from_millis(199)),
            true
        );
    }

    #[test]
    fn confirming_disarms_so_third_press_rearms() {
        let mut armer = EscInterruptArmer::default();
        let now = Instant::now();

        assert_eq!(armer.confirm_or_arm_at(now), false);
        assert_eq!(
            armer.confirm_or_arm_at(now + Duration::from_millis(100)),
            true
        );
        assert_eq!(
            armer.confirm_or_arm_at(now + Duration::from_millis(101)),
            false
        );
        assert_eq!(
            armer.confirm_or_arm_at(now + Duration::from_millis(102)),
            true
        );
    }

    #[test]
    fn press_after_window_rearms_without_confirming() {
        let mut armer = EscInterruptArmer::default();
        let now = Instant::now();

        assert_eq!(armer.confirm_or_arm_at(now), false);
        assert_eq!(
            armer.confirm_or_arm_at(now + Duration::from_millis(201)),
            false
        );
    }
}
