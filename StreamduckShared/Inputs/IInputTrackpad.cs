using System;
using Streamduck.Data;

namespace Streamduck.Inputs;

public interface IInputTrackpad {
	event Action<Int2>? TrackpadDragged;
}