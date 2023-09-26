using System;
using Streamduck.Devices;

namespace Streamduck.Api;

public abstract class Core : IDisposable {
	protected readonly Device _associatedDevice;

	protected Core(Device associatedDevice) {
		_associatedDevice = associatedDevice;
	}

	public void Dispose() {
		if (_associatedDevice is IDisposable disposable) disposable.Dispose();
		GC.SuppressFinalize(this);
	}

	public bool IsAlive() => _associatedDevice.Alive;
}