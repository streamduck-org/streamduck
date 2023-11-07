using System;
using Streamduck.Devices;
using Streamduck.Inputs;

namespace Streamduck.Cores;

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

	/**
	 * Push screen into the stack
	 */
	public abstract void PushScreen(Screen screen);

	/**
	 * Initialize a new screen with inputs and then push it into the stack
	 */
	public abstract void PushScreen(Func<Input[], Screen> pushFunction);

	/**
	 * Pops screen from the stack
	 */
	public abstract Screen? PopScreen();

	/**
	 * Replaces current screen with another
	 */
	public abstract Screen? ReplaceScreen(Screen newScreen);

	/**
	 * Initializes a new screen with inputs, then replaces current screen
	 */
	public abstract Screen? ReplaceScreen(Func<Input[], Screen> pushFunction);
	
	
	public abstract Screen? CurrentScreen { get; }
}