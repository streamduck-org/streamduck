namespace ElgatoStreamDeck.Inputs;

public abstract class Input {
	public abstract InputType Type { get; }
}

public enum InputType {
	ButtonStateChange,
	EncoderStateChange,
	EncoderTwist,
	TouchScreenPress,
	TouchScreenLongPress,
	TouchScreenSwipe
}