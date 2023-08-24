using System;

namespace ElgatoStreamDeck.Inputs;

public sealed class TouchScreenSwipe : Input {
	public override InputType Type => InputType.TouchScreenSwipe;

	public ushort StartX { get; init; }
	public ushort StartY { get; init; }

	public ushort EndX { get; init; }
	public ushort EndY { get; init; }
}

public static class TouchScreenSwipeExtension {
	public static TouchScreenSwipe? AsTouchScreenSwipe(this Input input) => input as TouchScreenSwipe;
}