// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Streamduck.Actions;
using Streamduck.Inputs;

namespace Streamduck.Triggers;

/**
 * Instance of Trigger that goes onto the input
 */
public abstract class TriggerInstance(Trigger original) {
	protected readonly List<ActionInstance> _actions = [];

	public Trigger Original { get; } = original;
	public IReadOnlyCollection<ActionInstance> Actions => _actions;

	public void AddAction(ActionInstance action) {
		_actions.Add(action);
	}

	public void RemoveAction(int index) {
		_actions.RemoveAt(index);
	}

	public abstract void Attach(Input input);
	public abstract void Detach(Input input);

	protected void InvokeActions() {
		Task.Run(async () => { await Task.WhenAll(_actions.Select(a => a.Invoke())); });
	}
}

/**
 * Instance of Trigger that goes onto the input, but with options
 */
public abstract class TriggerInstance<T>(Trigger original) : TriggerInstance(original) where T : class, new() {
	public T Options { get; set; } = new();
}