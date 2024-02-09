using System.Threading.Tasks;
using Streamduck.Inputs;
using Streamduck.Interfaces;

namespace Streamduck.Triggers;


public abstract class Trigger : INamed {
	public abstract string Name { get; }
	public abstract string? Description { get; }
	public abstract bool IsApplicableTo(Input input);
	public abstract Task<TriggerInstance> CreateInstance();
}