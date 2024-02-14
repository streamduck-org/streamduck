// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Collections.Generic;
using Streamduck.Devices;
using Streamduck.Inputs;

namespace Streamduck.Cores;

public abstract class Core(Device associatedDevice) : IDisposable {
	protected readonly Device _associatedDevice = associatedDevice;
	
	public DeviceIdentifier DeviceIdentifier { get; } = associatedDevice.Identifier;
	public IReadOnlyCollection<Input> Inputs { get; } = associatedDevice.Inputs;

	/**
	 * Top screen of the stack
	 */
	public abstract Screen? CurrentScreen { get; }
	
	/**
	 * Contents of the stack
	 */
	public abstract IEnumerable<Screen> ScreenStack { get; }

	public void Dispose() {
		if (_associatedDevice is IDisposable disposable) disposable.Dispose();
		GC.SuppressFinalize(this);
	}

	public bool IsAlive() => _associatedDevice.Alive;

	/**
	 * Create new screen that can later be pushed into the stack
	 */
	public abstract Screen NewScreen(bool canWrite = true);

	/**
	 * Push screen into the stack
	 */
	public abstract void PushScreen(Screen screen);

	/**
	 * Pops screen from the stack
	 */
	public abstract Screen? PopScreen();

	/**
	 * Replaces current screen with another
	 */
	public abstract Screen? ReplaceScreen(Screen newScreen);

	/**
	 * Called on every tick
	 */
	public abstract event Action? Tick;
}