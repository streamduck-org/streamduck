// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Collections.Generic;
using Streamduck.Cores;
using Streamduck.Devices;

namespace Streamduck.UI.Design;

public class ShellCore(Device associatedDevice) : Core(associatedDevice) {
	public override Screen? CurrentScreen { get; }
	public override IEnumerable<Screen> ScreenStack { get; }
	public override Screen NewScreen(bool canWrite = true) => throw new NotImplementedException();

	public override void PushScreen(Screen screen) {
		throw new NotImplementedException();
	}

	public override Screen? PopScreen() => throw new NotImplementedException();

	public override Screen? ReplaceScreen(Screen newScreen) => throw new NotImplementedException();

	public override event Action? Tick;
}