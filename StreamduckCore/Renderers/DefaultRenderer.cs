// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using SixLabors.ImageSharp;
using Streamduck.Cores;
using Streamduck.Rendering;

namespace StreamduckCore.Renderers;

public class DefaultRenderer : Renderer<DefaultRenderer.Settings> {
	public override string Name => "Default Renderer";
	public override long Hash(ScreenItem input, Settings renderConfig) => throw new NotImplementedException();
	public override Image Render(ScreenItem input, Settings renderConfig) => throw new NotImplementedException();

	public class Settings { }
}