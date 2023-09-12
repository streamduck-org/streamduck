using Streamduck.Definitions.Inputs;

namespace Streamduck.Definitions.Scripting; 

/**
 * Instance of a script that gets put onto an input
 */
public abstract class ScriptInstance {
	protected readonly Script _script;
	
	protected ScriptInstance(Script script) {
		_script = script;
	}

	/**
	 * Should add all the necessary subscribers to the input
	 */
	public abstract void AddHooks(Input input);
	
	/**
	 * Should remove all the subscribers that the instance had to the input
	 */
	public abstract void RemoveHooks(Input input);
}