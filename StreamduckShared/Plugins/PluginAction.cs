using System.Threading.Tasks;
using Streamduck.Interfaces;

namespace Streamduck.Plugins; 

/**
 * Indicates something that can be called by Scripting System
 */
public abstract class PluginAction : INamed {
	public abstract string Name { get; }
	
	public abstract string? Description { get; }

	/**
	 * <exception cref="System.ArgumentException">If arguments were of invalid type</exception>
	 */
	public abstract Task Invoke();
}