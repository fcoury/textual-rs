"""
Python Textual equivalent of the Rust switch example.

This demonstrates the same functionality:
- Two toggle switches (WiFi and Bluetooth)
- Focus navigation with Tab/Arrow keys
- Type-safe message handling via Switch.Changed events
"""

from textual.app import App, ComposeResult
from textual.containers import Center, Middle, Vertical
from textual.widgets import Switch, Static
from textual import on


class LabeledSwitch(Static):
    """A switch with a label."""

    def __init__(self, label: str, switch_id: str, value: bool = False) -> None:
        super().__init__()
        self.label = label
        self.switch_id = switch_id
        self.initial_value = value

    def compose(self) -> ComposeResult:
        yield Static(self.label, classes="label")
        yield Switch(value=self.initial_value, id=self.switch_id)


class SwitchApp(App):
    """A simple app with two toggle switches."""

    CSS = """
    Screen {
        align: center middle;
    }

    Vertical {
        width: auto;
        height: auto;
    }

    LabeledSwitch {
        layout: horizontal;
        width: auto;
        height: 3;
        margin: 1;
    }

    LabeledSwitch .label {
        width: 12;
        content-align: right middle;
        padding-right: 1;
    }

    Switch {
        width: auto;
    }
    """

    BINDINGS = [
        ("q", "quit", "Quit"),
    ]

    def __init__(self) -> None:
        super().__init__()
        self.wifi_on = False
        self.bt_on = False

    def compose(self) -> ComposeResult:
        with Middle():
            with Center():
                with Vertical():
                    yield LabeledSwitch("WiFi", "wifi", self.wifi_on)
                    yield LabeledSwitch("Bluetooth", "bluetooth", self.bt_on)

    # Type-safe event handling - the Switch.Changed event carries the switch and value
    @on(Switch.Changed, "#wifi")
    def handle_wifi_changed(self, event: Switch.Changed) -> None:
        self.wifi_on = event.value
        self.log(f"WiFi: {'ON' if self.wifi_on else 'OFF'}")

    @on(Switch.Changed, "#bluetooth")
    def handle_bluetooth_changed(self, event: Switch.Changed) -> None:
        self.bt_on = event.value
        self.log(f"Bluetooth: {'ON' if self.bt_on else 'OFF'}")


if __name__ == "__main__":
    app = SwitchApp()
    app.run()
