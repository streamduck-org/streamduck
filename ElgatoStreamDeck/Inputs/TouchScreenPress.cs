using System;

namespace ElgatoStreamDeck.Inputs;

public sealed class TouchScreenPress : Input {
	public override InputType Type => InputType.TouchScreenPress;

	public ushort X { get; init; }
	public ushort Y { get; init; }
}

public static class TouchScreenPressExtension {
	public static TouchScreenPress? AsTouchScreenPress(this Input input) => input as TouchScreenPress;
}