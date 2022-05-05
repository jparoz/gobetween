use super::button::Button;

#[derive(Debug, Clone)]
pub enum Message {
    ButtonPressed(Button),
    ButtonReleased(Button),
}
