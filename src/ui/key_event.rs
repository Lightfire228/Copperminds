use iced::keyboard::{Key, Location, Modifiers, key};
use smol_str::SmolStr;


#[allow(unused)]
#[derive(Debug)]
/// This is a copy of `iced::keyboard::Event::KeyPressed {}`
pub struct KeyPressed {
    /// The key pressed.
    pub key: Key,

    /// The key pressed with all keyboard modifiers applied, except Ctrl.
    pub modified_key: Key,

    /// The physical key pressed.
    pub physical_key: key::Physical,

    /// The location of the key.
    pub location: Location,

    /// The state of the modifier keys.
    pub modifiers: Modifiers,

    /// The text produced by the key press, if any.
    pub text: Option<SmolStr>,

    /// Whether the event was the result of key repeat.
    pub repeat: bool,
}


use iced::keyboard::Event;

impl TryFrom<Event> for KeyPressed {
    type Error = ();

    // this is silly
    fn try_from(value: Event) -> Result<KeyPressed, Self::Error> {

        match value {
            Event::KeyPressed {
                key,
                modified_key,
                physical_key,
                location,
                modifiers,
                text,
                repeat,
            } => Ok(KeyPressed {
                key,
                modified_key,
                physical_key,
                location,
                modifiers,
                text,
                repeat,
            }),
            _ => Err(())
        }
    }
}
