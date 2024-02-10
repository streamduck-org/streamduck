// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Threading.Tasks;
using Streamduck.Interfaces;

namespace Streamduck.Plugins.Methods;

public class ConfigurableReflectedAction<T, C>(
	string name,
	Func<T, C, Task> actionToCall,
	string? description = null)
	: PluginAction<T>, IConfigurable<C> where T : class, new() where C : class, new() {
	public override string Name { get; } = name;
	public override string? Description { get; } = description;
	public C Config { get; set; } = new();
	public override Task Invoke(T data) => actionToCall.Invoke(data, Config);
}