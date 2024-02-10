// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using Streamduck.Data;

namespace Streamduck.Inputs;

public interface IInputTouchScreen {
	event Action<Int2>? TouchScreenPressed;
	event Action<Int2>? TouchScreenReleased;

	public interface Drag {
		event Action<Int2>? TouchScreenDragStart;
		event Action<Int2>? TouchScreenDragEnd;
	}

	public interface Dragging {
		event Action<Int2>? TouchScreenDragging;
	}

	public interface Hover {
		event Action<Int2>? TouchScreenHoverStart;
		event Action<Int2>? TouchScreenHovering;
		event Action<Int2>? TouchScreenHoverEnd;
	}
}