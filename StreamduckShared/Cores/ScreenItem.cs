using System.Collections.Generic;
using Streamduck.Inputs;
using Streamduck.Plugins;
using Streamduck.Triggers;

namespace Streamduck.Cores; 

/**
 * Item of the screen that has optional renderer settings and can contain actions
 */
public abstract class ScreenItem {
	public interface IRenderable {
		NamespacedName? RendererName { get; set; }
		object? RendererSettings { get; set; }
	}
	
	public interface IRenderable<T> : IRenderable where T : class, new() {
		object? IRenderable.RendererSettings {
			get => RendererSettings;
			set {
				if (value is T casted) {
					RendererSettings = casted;
				}
			}
		}

		new T? RendererSettings { get; set; }
	}
	
	public abstract IEnumerable<TriggerInstance> Triggers { get; }

	public abstract void AddTrigger(TriggerInstance trigger, bool attachToInput = true);

	public abstract bool RemoveTrigger(TriggerInstance trigger);

	public abstract void Attach(Input input);
	
	public abstract void Detach();
}

