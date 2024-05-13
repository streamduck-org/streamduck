// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Collections.Generic;
using System.Linq;
using NLog;
using Streamduck.Devices;
using Streamduck.Plugins;

namespace Streamduck.Cores;

public sealed class CoreImpl : Core {
	private static readonly Logger _l = LogManager.GetCurrentClassLogger();

	private readonly IPluginQuery _pluginQuery;

	internal readonly Stack<Screen> _screenStack;

	public CoreImpl(Device device, NamespacedDeviceIdentifier deviceIdentifier, IPluginQuery pluginQuery) :
		base(device) {
		DeviceIdentifier = deviceIdentifier;
		_pluginQuery = pluginQuery;
		_screenStack = new Stack<Screen>();
		
		PushScreen(NewScreen("Root"));
		PushScreen(NewScreen("Item1"));
		PushScreen(NewScreen("Item2"));
		PushScreen(NewScreen("Item3"));
		PushScreen(NewScreen("Item1"));
		PushScreen(NewScreen("Item2"));
		PushScreen(NewScreen("Item3"));
		PushScreen(NewScreen("Item1"));
		PushScreen(NewScreen("Item2"));
		PushScreen(NewScreen("Item3"));
		PushScreen(NewScreen("Item1"));
		PushScreen(NewScreen("Item2"));
		PushScreen(NewScreen("Item3"));
		PushScreen(NewScreen("Item1"));
		PushScreen(NewScreen("Item2"));
		PushScreen(NewScreen("Item3"));
		PushScreen(NewScreen("Item1"));
		PushScreen(NewScreen("Item2"));
		PushScreen(NewScreen("Item3"));
		PushScreen(NewScreen("Item1"));
		PushScreen(NewScreen("Item2"));
		PushScreen(NewScreen("Item3"));
		PushScreen(NewScreen("Item1"));
		PushScreen(NewScreen("Item2"));
		PushScreen(NewScreen("Item3"));
        
		
		device.Died += () => _l.Warn("Device {} died", DeviceIdentifier);
	}

	public override NamespacedDeviceIdentifier DeviceIdentifier { get; }

	public override Screen? CurrentScreen {
		get {
			lock (_screenStack) {
				if (_screenStack.TryPeek(out var screen)) return screen;
			}

			return null;
		}
	}

	internal ScreenImpl? CurrentScreenImpl => CurrentScreen as ScreenImpl;

	public override IEnumerable<Screen> ScreenStack {
		get {
			lock (_screenStack) {
				return _screenStack;
			}
		}
	}

	public override Screen NewScreen(string name = "Screen", bool canWrite = true) =>
		new ScreenImpl(this, _pluginQuery, _associatedDevice.Inputs) {
			Name = name,
			CanWrite = canWrite
		};

	public override void PushScreen(Screen screen) {
		lock (_screenStack) {
			_screenStack.TryPeek(out var oldScreen);
			(oldScreen as ScreenImpl)?.DetachFromInputs();
			_screenStack.Push(screen);
			(screen as ScreenImpl)?.AttachToInputs();
		}
	}

	public override Screen? PopScreen() {
		lock (_screenStack) {
			if (_screenStack.Count <= 1) return null;
			_screenStack.TryPop(out var screen);
			(screen as ScreenImpl)?.DetachFromInputs();
			if (_screenStack.TryPeek(out var newScreen)) (newScreen as ScreenImpl)?.AttachToInputs();
			return screen;
		}
	}

	public override Screen? ReplaceScreen(Screen newScreen) {
		lock (_screenStack) {
			_screenStack.TryPop(out var screen);
			(screen as ScreenImpl)?.DetachFromInputs();
			_screenStack.Push(newScreen);
			(newScreen as ScreenImpl)?.AttachToInputs();
			return screen;
		}
	}

	public override event Action? Tick;

	internal void CallTick() {
		Tick?.Invoke();
	}
}