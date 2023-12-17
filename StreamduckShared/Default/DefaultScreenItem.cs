using Streamduck.Cores;
using Streamduck.Plugins;

namespace Streamduck.Default; 

public class DefaultScreenItem : ScreenItem, ScreenItem.IRenderable {
	public NamespacedName? RendererName { get; set; }
	public object? RendererSettings { get; set; }
}