using System.Collections.Generic;
using Streamduck.Plugins;
using Streamduck.Scripting;

namespace Streamduck.Cores; 

/**
 * Item of the screen that can contain scripts
 */
public abstract class ScreenItem {
	public interface IRenderable {
		NamespacedName? RendererName { get; set; }
		object? RendererSettings { get; set; }
	}
	
	public interface IRenderable<T> : IRenderable where T : class {
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
	
	public abstract List<ScriptInstance> Scripts { get; }
}

