using System;
using System.Threading.Tasks;
using Streamduck.Inputs;

namespace Streamduck.Triggers;

/**
 * Instance of Trigger that goes onto the input
 */
public abstract class TriggerInstance(Trigger original) {
	public Trigger Original { get; } = original;
	public abstract event Action? Triggered;
	
	public abstract void Attach(Input input);
	public abstract void Detach(Input input);
}

/**
 * Instance of Trigger that goes onto the input, but with options
 */
public abstract class TriggerInstance<T>(Trigger original) : TriggerInstance(original) where T : class, new() {
	public T Options { get; set; } = new();
}