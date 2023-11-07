using System.Collections.Generic;
using Streamduck.Plugins;
using Streamduck.Scripting;

namespace Streamduck.Cores.Default; 

public class DefaultScreenItem : ScreenItem, ScreenItem.IRenderable {
	public override List<ScriptInstance> Scripts { get; } = new();
	public NamespacedName? RendererName { get; set; }
	public object? RendererSettings { get; set; }
}