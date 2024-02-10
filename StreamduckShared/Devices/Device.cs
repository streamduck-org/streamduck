// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

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