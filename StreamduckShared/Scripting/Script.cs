using System.Threading.Tasks;

namespace Streamduck.Scripting;

/**
 * The actual script that runs things
 */
public abstract class Script {
	public abstract ScriptInstance MakeInstance();

	/**
	 * Should serialize script content into a form that ScriptingSystem could then Deserialize with
	 */
	public abstract Task<byte[]> Serialize();
}