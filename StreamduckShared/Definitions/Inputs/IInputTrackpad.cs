using System;

namespace Streamduck.Definitions.Inputs;

public interface IInputTrackpad {
	event Action<Int2>? TrackpadDragged;
}