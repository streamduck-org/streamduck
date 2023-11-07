using System.Collections.Generic;
using System.Linq;
using Streamduck.Cores;
using Streamduck.Inputs;

namespace Streamduck.Utils; 

public static class ScreenExtensions {
	public static void AddHooks(this Screen screen, IEnumerable<Input> inputs) {
		foreach (var (item, input) in screen.Items.Zip(inputs)) {
			if (item is null) continue;
			foreach (var script in item.Scripts) {
				script.AddHooks(input);
			}
		}
	}
	
	public static void RemoveHooks(this Screen screen, IEnumerable<Input> inputs) {
		foreach (var (item, input) in screen.Items.Zip(inputs)) {
			if (item is null) continue;
			foreach (var script in item.Scripts) {
				script.RemoveHooks(input);
			}
		}
	}
}