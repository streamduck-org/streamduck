using Streamduck.Api;
using Streamduck.Scripting;

namespace Streamduck.Plugins; 

/**
 * Indicates something that can be called by Scripting System
 */
public abstract class PluginAction : INamed {
	public abstract string Name { get; }
	
	public abstract string? Description { get; }

	public abstract DataInfo[] Parameters { get; }

	/**
	 * <exception cref="System.ArgumentException">If arguments were of invalid type</exception>
	 */
	public abstract void Invoke(object[] arguments);
}