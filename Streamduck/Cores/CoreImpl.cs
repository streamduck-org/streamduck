using System;
using System.Collections.Generic;
using NLog;
using Streamduck.Api;
using Streamduck.Devices;
using Streamduck.Inputs;
using Streamduck.Utils;

namespace Streamduck.Cores;

public class CoreImpl : Core {
	private static readonly Logger _l = LogManager.GetCurrentClassLogger();
	protected readonly NamespacedDeviceIdentifier _deviceIdentifier;

	private readonly Stack<Screen> _screenStack = new();

	public CoreImpl(Device device, NamespacedDeviceIdentifier deviceIdentifier) : base(device) {
		_deviceIdentifier = deviceIdentifier;
		device.Died += () => _l.Warn("Device {} died", _deviceIdentifier);
	}

	private void SwapHooks(Screen? oldScreen, Screen? newScreen) {
		oldScreen?.RemoveHooks(_associatedDevice.Inputs);
		newScreen?.AddHooks(_associatedDevice.Inputs);
	}

	public override void PushScreen(Screen screen) {
		lock (_screenStack) {
			_screenStack.TryPeek(out var oldScreen);
			_screenStack.Push(screen);
			
			SwapHooks(oldScreen, screen);
		}
	}

	public override void PushScreen(Func<Input[], Screen> pushFunction) {
		lock (_screenStack) {
			_screenStack.TryPeek(out var oldScreen);
			var newScreen = pushFunction.Invoke(_associatedDevice.Inputs);
			_screenStack.Push(newScreen);
			
			SwapHooks(oldScreen, newScreen);
		}
	}

	public override Screen? PopScreen() {
		lock (_screenStack) {
			if (!_screenStack.TryPop(out var screen)) return null;
			
			_screenStack.TryPeek(out var newScreen);
			SwapHooks(screen, newScreen);
			return screen;
		}
	}

	public override Screen? ReplaceScreen(Screen newScreen) {
		lock (_screenStack) {
			_screenStack.TryPop(out var screen);
			_screenStack.Push(newScreen);
			SwapHooks(screen, newScreen);
			return screen;
		}
	}

	public override Screen? ReplaceScreen(Func<Input[], Screen> pushFunction) {
		lock (_screenStack) {
			_screenStack.TryPop(out var screen);
			var newScreen = pushFunction.Invoke(_associatedDevice.Inputs);
			_screenStack.Push(newScreen);
			SwapHooks(screen, newScreen);
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
}