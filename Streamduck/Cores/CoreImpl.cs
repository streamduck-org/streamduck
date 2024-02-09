using System;
using System.Collections.Generic;
using NLog;
using Streamduck.Devices;
using Streamduck.Inputs;
using Streamduck.Plugins;

namespace Streamduck.Cores;

public class CoreImpl : Core {
	private static readonly Logger _l = LogManager.GetCurrentClassLogger();
	protected readonly NamespacedDeviceIdentifier _deviceIdentifier;

	private readonly Stack<Screen> _screenStack = new();

	protected readonly IPluginQuery _pluginQuery;

	public CoreImpl(Device device, NamespacedDeviceIdentifier deviceIdentifier, IPluginQuery pluginQuery) : base(device) {
		_deviceIdentifier = deviceIdentifier;
		_pluginQuery = pluginQuery;
		device.Died += () => _l.Warn("Device {} died", _deviceIdentifier);
	}

	public override Screen NewScreen(bool canWrite = true) =>
		new ScreenImpl(_associatedDevice.Inputs, _pluginQuery) {
			CanWrite = canWrite
		};

	public override void PushScreen(Screen screen) {
		lock (_screenStack) {
			_screenStack.TryPeek(out var oldScreen);
			_screenStack.Push(screen);
		}
	}

	public override void PushScreen(Func<Input[], Screen> pushFunction) {
		lock (_screenStack) {
			_screenStack.TryPeek(out var oldScreen);
			var newScreen = pushFunction.Invoke(_associatedDevice.Inputs);
			_screenStack.Push(newScreen);
		}
	}

	public override Screen? PopScreen() {
		lock (_screenStack) {
			return !_screenStack.TryPop(out var screen) ? null : screen;
		}
	}

	public override Screen? ReplaceScreen(Screen newScreen) {
		lock (_screenStack) {
			_screenStack.TryPop(out var screen);
			_screenStack.Push(newScreen);
			return screen;
		}
	}

	public override Screen? ReplaceScreen(Func<Input[], Screen> pushFunction) {
		lock (_screenStack) {
			_screenStack.TryPop(out var screen);
			var newScreen = pushFunction.Invoke(_associatedDevice.Inputs);
			_screenStack.Push(newScreen);
			return screen;
		}
	}

	public override Screen? CurrentScreen {
		get {
			lock (_screenStack) {
				if (_screenStack.TryPeek(out var screen)) {
					return screen;
				}
			}

			return null;
		}
	}

	public override event Action? Tick;

	internal void CallTick() {
		Tick?.Invoke();
	}
}