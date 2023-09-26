using System;
using Streamduck.Inputs;

namespace Streamduck.Devices;

public abstract class Device {
	protected Device(DeviceIdentifier identifier) {
		Identifier = identifier;
		Alive = true;
		Died += () => Alive = false;
	}

	public bool Alive { get; private set; }

	public bool Busy { get; set; }
	public DeviceIdentifier Identifier { get; }

	public abstract Input[] Inputs { get; }

	public event Action Died;

	protected void Die() {
		if (Alive) Died.Invoke();
	}

	public void ThrowDisconnectedIfDead() {
		if (!Alive) throw new DeviceDisconnectedException();
	}
}