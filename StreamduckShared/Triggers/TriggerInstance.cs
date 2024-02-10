// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
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