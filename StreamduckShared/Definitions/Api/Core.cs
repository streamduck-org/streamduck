using System;
using Streamduck.Definitions.Devices;

namespace Streamduck.Definitions.Api;

public abstract class Core : IDisposable {
	protected readonly Device _associatedDevice;

	protected Core(Device associatedDevice) {
		_associatedDevice = associatedDevice;
	}

	public bool IsAlive() => _associatedDevice.Alive;
	
	public void Dispose() {
		if (_associatedDevice is IDisposable disposable) {
			disposable.Dispose();
		}
		GC.SuppressFinalize(this);
	}
}