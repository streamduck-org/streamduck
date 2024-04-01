// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Collections.Generic;
using Streamduck.Inputs;
using Streamduck.Plugins;
using Streamduck.Triggers;

namespace Streamduck.Cores;

/**
 * Item of the screen that has optional renderer settings and can contain actions
 */
public abstract class ScreenItem {
	public abstract IReadOnlyCollection<TriggerInstance> Triggers { get; }

	public abstract void AddTrigger(TriggerInstance trigger, bool attachToInput = true);

	public abstract bool RemoveTrigger(TriggerInstance trigger);

	public abstract void Attach(Input input);

	public abstract void Detach();

	public interface IRenderable {
		NamespacedName? RendererName { get; set; }
		object? RendererSettings { get; set; }
	}
}