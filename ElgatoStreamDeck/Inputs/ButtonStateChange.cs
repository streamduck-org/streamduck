using System;

namespace ElgatoStreamDeck.Inputs;

public sealed class ButtonStateChange : Input {
	public override InputType Type => InputType.ButtonStateChange;

	public bool[] Buttons { get; init; } = Array.Empty<bool>();
}

public static class ButtonStateChangeExtension {
	public static ButtonStateChange? AsButtonStateChange(this Input input) => input as ButtonStateChange;
}