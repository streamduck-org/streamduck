using System;

namespace Streamduck.Definitions.Inputs;

public interface IInputTouchScreen {
	event Action<Int2>? TouchScreenPressed;
	event Action<Int2>? TouchScreenReleased;

	public interface Drag {
		event Action<Int2>? TouchScreenDragStart;
		event Action<Int2>? TouchScreenDragging;
		event Action<Int2>? TouchScreenDragEnd;
	}

	public interface Hover {
		event Action<Int2>? TouchScreenHoverStart;
		event Action<Int2>? TouchScreenHovering;
		event Action<Int2>? TouchScreenHoverEnd;
	}
}