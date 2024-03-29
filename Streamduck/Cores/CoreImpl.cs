// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Collections.Generic;
using NLog;
using Streamduck.Devices;
using Streamduck.Plugins;

namespace Streamduck.Cores;

public class CoreImpl : Core {
	private static readonly Logger _l = LogManager.GetCurrentClassLogger();
	protected readonly NamespacedDeviceIdentifier _deviceIdentifier;

	protected readonly IPluginQuery _pluginQuery;

	private readonly Stack<Screen> _screenStack = new();

	public CoreImpl(Device device, NamespacedDeviceIdentifier deviceIdentifier, IPluginQuery pluginQuery) :
		base(device) {
		_deviceIdentifier = deviceIdentifier;
		_pluginQuery = pluginQuery;
		device.Died += () => _l.Warn("Device {} died", _deviceIdentifier);
	}

	public override Screen? CurrentScreen {
		get {
			lock (_screenStack) {
				if (_screenStack.TryPeek(out var screen)) return screen;
			}

			return null;
		}
	}

	public override IEnumerable<Screen> ScreenStack {
		get {
			lock (_screenStack) {
				return _screenStack;
			}
		}
	}

	public override Screen NewScreen(bool canWrite = true) =>
		new ScreenImpl(this, _associatedDevice.Inputs, _pluginQuery) {
			CanWrite = canWrite
		};

	public override void PushScreen(Screen screen) {
		lock (_screenStack) {
			_screenStack.TryPeek(out var oldScreen);
			oldScreen?.DetachFromInputs();
			_screenStack.Push(screen);
			screen.AttachToInputs();
		}
	}

	public override Screen? PopScreen() {
		lock (_screenStack) {
			_screenStack.TryPop(out var screen);
			screen?.DetachFromInputs();
			if (_screenStack.TryPeek(out var newScreen)) newScreen.AttachToInputs();
			return screen;
		}
	}

	public override Screen? ReplaceScreen(Screen newScreen) {
		lock (_screenStack) {
			_screenStack.TryPop(out var screen);
			screen?.DetachFromInputs();
			_screenStack.Push(newScreen);
			newScreen.AttachToInputs();
			return screen;
		}
	}

	public override event Action? Tick;

	internal void CallTick() {
		Tick?.Invoke();
	}
}