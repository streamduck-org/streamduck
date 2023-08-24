using System;

namespace ElgatoStreamDeck.Inputs;

public sealed class TouchScreenLongPress : Input {
	public override InputType Type => InputType.TouchScreenLongPress;

	public ushort X { get; init; }
	public ushort Y { get; init; }
}

public static class TouchScreenLongPressExtension {
	public static TouchScreenLongPress? AsTouchScreenLongPress(this Input input) => input as TouchScreenLongPress;
}