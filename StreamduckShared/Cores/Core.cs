using System;
using Streamduck.Devices;
using Streamduck.Inputs;

namespace Streamduck.Cores;

public abstract class Core(Device associatedDevice) : IDisposable {
	protected readonly Device _associatedDevice = associatedDevice;

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
	
	/**
	 * Top screen of the stack
	 */
	public abstract Screen? CurrentScreen { get; }
	
	/**
	 * Called on every tick
	 */
	public abstract event Action? Tick;
}