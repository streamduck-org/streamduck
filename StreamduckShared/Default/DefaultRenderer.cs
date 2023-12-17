using SixLabors.ImageSharp;
using Streamduck.Cores;
using Streamduck.Rendering;

namespace Streamduck.Default; 

public class DefaultRenderer : Renderer<DefaultRenderer.Settings> {
	public class Settings {
		
	}
	
	public override string Name => "Default Renderer";
	public override long Hash(ScreenItem input, Settings renderConfig) => throw new System.NotImplementedException();
	public override Image Render(ScreenItem input, Settings renderConfig) => throw new System.NotImplementedException();
}