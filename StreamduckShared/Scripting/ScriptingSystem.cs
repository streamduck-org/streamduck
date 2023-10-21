using System.Threading.Tasks;
using Avalonia.Controls;
using Streamduck.Api;

namespace Streamduck.Scripting;

public abstract class ScriptingSystem : INamed {
	public abstract string Name { get; }
	public abstract Task<Script> New();

	/**
	 * Provides graphical editor for the script
	 */
	public abstract Control Editor(Script script);
	public abstract Task<Script> Deserialize(byte[] data);
}