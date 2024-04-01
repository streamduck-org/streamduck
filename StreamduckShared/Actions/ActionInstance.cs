// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Threading.Tasks;

namespace Streamduck.Actions;

public class ActionInstance(PluginAction original) {
	public PluginAction Original { get; } = original;
	public object? Data { get; set; }

	public async Task Invoke() {
		Data ??= await Original.DefaultData();
		await Original.Invoke(Data);
	}
}