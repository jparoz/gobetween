use super::button::Button;
use super::encoder::Encoder;
use super::fader::Fader;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Message {
    ButtonPressed(Button),
    ButtonReleased(Button),
    FaderLevel(Fader, u16),
    EncoderRotate(Encoder, i8),
}
