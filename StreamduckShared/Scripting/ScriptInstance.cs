using System;
using Streamduck.Inputs;

namespace Streamduck.Scripting;

/**
 * Instance of a script that gets put onto an input
 */
public abstract class ScriptInstance {
	protected readonly WeakReference<Script> _script;

	protected ScriptInstance(Script script) {
		_script = new WeakReference<Script>(script);
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