| method | setup | handler | pure (memo) | after dispose | other App | dropped App | borrowed |
|---|---|---|---|---|---|---|---|
| `Signal::get` | ok=1 | ok=1 | ok=1 | err:Disposed | err:WrongApp | err:WrongApp | ok=1 |
| `Signal::get_untracked` | ok=1 | ok=1 | ok=1 | err:Disposed | err:WrongApp | err:WrongApp | ok=1 |
| `Signal::with` | ok=1 | ok=1 | ok=1 | err:Disposed | err:WrongApp | err:WrongApp | ok=1 |
| `Signal::with_untracked` | ok=1 | ok=1 | ok=1 | err:Disposed | err:WrongApp | err:WrongApp | ok=1 |
| `Signal::set` | ok | ok | err:WriteInPureContext | ign+diag:WriteAfterDispose | err:WrongApp | err:WrongApp | err:BorrowedWrite |
| `Signal::update` | ok | ok | err:WriteInPureContext | err:WriteAfterDispose | err:WrongApp | err:WrongApp | err:BorrowedWrite |
| `Signal::notify` | ok | ok | err:WriteInPureContext | ign+diag:WriteAfterDispose | err:WrongApp | err:WrongApp | err:BorrowedWrite |
| `Signal::field` | ok | err:SetupOnly | err:SetupOnly | err:SetupOnly | err:WrongApp | err:WrongApp | err:SetupOnly |
| `Signal::try_get` | ok=Some(1) | ok=Some(1) | ok=Some(1) | ok=None | ok=None | ok=None | ok=Some(1) |
| `Signal::into_value` | ok | ok | ok | ok | ok | ok | ok |
| `Value::get` | ok=2 | ok=2 | ok=2 | err:Disposed | err:WrongApp | err:WrongApp | ok=2 |
| `Value::get_untracked` | ok=2 | ok=2 | ok=2 | err:Disposed | err:WrongApp | err:WrongApp | ok=2 |
| `Value::with` | ok=2 | ok=2 | ok=2 | err:Disposed | err:WrongApp | err:WrongApp | ok=2 |
| `Value::with_untracked` | ok=2 | ok=2 | ok=2 | err:Disposed | err:WrongApp | err:WrongApp | ok=2 |
| `Value::try_get` | ok=Some(2) | ok=Some(2) | ok=Some(2) | ok=None | ok=None | ok=None | ok=Some(2) |
| `IntoValue::into_value (constant)` | ok | err:SetupOnly | err:SetupOnly | err:SetupOnly | err:SetupOnly | err:SetupOnly | - |
| `Var::get` | ok=0 | ok=0 | ok=0 | err:Disposed | err:WrongApp | err:WrongApp | ok=0 |
| `Var::with` | ok=0 | ok=0 | ok=0 | err:Disposed | err:WrongApp | err:WrongApp | ok=0 |
| `Var::set` | ok | ok | ok | err:Disposed | err:WrongApp | err:WrongApp | err:BorrowedWrite |
| `Var::update` | ok | ok | ok | err:Disposed | err:WrongApp | err:WrongApp | err:BorrowedWrite |
| `Callback::call` | ok | ok | ok+err:WriteInPureContext | err:Disposed | err:WrongApp | err:WrongApp | ok+err:BorrowedWrite |
| `Callback::try_call` | ok=Ok(()) | ok=Ok(()) | ok=Err("WriteInPureContext") | ok=Err("Disposed") | ok=Err("WrongApp") | ok=Err("WrongApp") | ok=Err("BorrowedWrite") |
| `Callback::bind` | ok | err:SetupOnly | err:SetupOnly | err:SetupOnly | err:WrongApp | err:WrongApp | - |
| `Callback::to_fn` | ok | ok | ok+err:WriteInPureContext | err:Disposed | err:WrongApp | err:WrongApp | ok+err:BorrowedWrite |
| `Binding::get` | ok=2 | ok=2 | ok=2 | err:Disposed | err:WrongApp | err:WrongApp | ok=2 |
| `Binding::with` | ok=2 | ok=2 | ok=2 | err:Disposed | err:WrongApp | err:WrongApp | ok=2 |
| `Binding::set` | ok | ok | ok+err:WriteInPureContext | err:Disposed | err:WrongApp | err:WrongApp | ok |
| `binding` | ok | err:SetupOnly | err:SetupOnly | err:SetupOnly | err:SetupOnly | err:SetupOnly | - |
| `Selector::is` | ok=true | ok=true | ok=true | err:Disposed | err:WrongApp | err:WrongApp | ok=true |
| `NodeRef::focus` | ok+err:Platform | ok | err:WriteInPureContext | err:Disposed | err:WrongApp | err:WrongApp | - |
| `NodeRef::scroll_into_view` | ok | ok | err:WriteInPureContext | err:Disposed | err:WrongApp | err:WrongApp | - |
| `NodeRef::rect` | ok | ok | ok | err:Disposed | err:WrongApp | err:WrongApp | - |
| `NodeRef::focused` | ok | err:SetupOnly | err:SetupOnly | err:Disposed | err:WrongApp | err:WrongApp | - |
| `NodeRef::focus_within` | ok | err:SetupOnly | err:SetupOnly | err:Disposed | err:WrongApp | err:WrongApp | - |
| `Resource::latest` | ok=None | ok=Some(1) | ok=Some(1) | err:Disposed | err:WrongApp | err:WrongApp | - |
| `Resource::refetch` | ok | ok | err:WriteInPureContext | err:Disposed | err:WrongApp | err:WrongApp | - |
| `Resource::cancel` | ok | ok | err:WriteInPureContext | err:Disposed | err:WrongApp | err:WrongApp | - |
| `Resource::state` | ok=Pending | ok=Ready(1) | ok=Ready(1) | err:Disposed | err:WrongApp | err:WrongApp | - |
| `Action::run` | ok | ok | err:WriteInPureContext | err:Disposed | err:WrongApp | err:WrongApp | - |
| `Action::retry` | ok | ok | err:WriteInPureContext | err:Disposed | err:WrongApp | err:WrongApp | - |
| `Action::cancel` | ok | ok | err:WriteInPureContext | err:Disposed | err:WrongApp | err:WrongApp | - |
| `Action::state` | ok=Idle | ok=Idle | ok=Idle | err:Disposed | err:WrongApp | err:WrongApp | - |
| `Action::latest` | ok=None | ok=None | ok=None | err:Disposed | err:WrongApp | err:WrongApp | - |
| `Action::input` | ok=None | ok=None | ok=None | err:Disposed | err:WrongApp | err:WrongApp | - |
| `Action::attempt` | ok=0 | ok=0 | ok=0 | err:Disposed | err:WrongApp | err:WrongApp | - |
| `Action::on_done` | ok | err:SetupOnly | err:SetupOnly | err:Disposed | err:WrongApp | err:WrongApp | - |
| `Action::policy` | ok | err:SetupOnly | err:SetupOnly | err:Disposed | err:WrongApp | err:WrongApp | - |
| `Action::run (Queue)` | ok | ok | err:WriteInPureContext | err:Disposed | err:WrongApp | err:WrongApp | - |
| `Action::state (Queue)` | ok | ok | ok | err:Disposed | err:WrongApp | err:WrongApp | - |
| `Action::pending_len (Queue)` | ok=0 | ok=0 | ok=0 | err:Disposed | err:WrongApp | err:WrongApp | - |
| `Action::on_done (Queue)` | ok | err:SetupOnly | err:SetupOnly | err:Disposed | err:WrongApp | err:WrongApp | - |
| `Action::close (Queue)` | ok | ok | err:WriteInPureContext | err:Disposed | err:WrongApp | err:WrongApp | - |
| `Task::cancel` | ok | ok | err:WriteInPureContext | ok | err:WrongApp | err:WrongApp | - |
| `Task::is_finished` | ok=false | ok=true | ok=true | ok=true | err:WrongApp | err:WrongApp | - |
| `FrameLoop::start` | ok | ok | err:WriteInPureContext | err:Disposed | err:WrongApp | err:WrongApp | - |
| `FrameLoop::stop` | ok | ok | err:WriteInPureContext | err:Disposed | err:WrongApp | err:WrongApp | - |
| `FrameLoop::rate` | ok | ok | ok | err:Disposed | err:WrongApp | err:WrongApp | - |
| `Animated::get` | ok=2 | ok=2 | ok=2 | err:Disposed | err:WrongApp | err:WrongApp | ok=2 |
| `Animated::get_untracked` | ok=2 | ok=2 | ok=2 | err:Disposed | err:WrongApp | err:WrongApp | ok=2 |
| `Animated::with` | ok=2 | ok=2 | ok=2 | err:Disposed | err:WrongApp | err:WrongApp | ok=2 |
| `Animated::value` | ok | ok | ok | ok | ok | ok | ok |
| `Animated::jump_to` | ok | ok | err:WriteInPureContext | err:Disposed | err:WrongApp | err:WrongApp | err:BorrowedWrite |
| `Animated::is_running` | ok=false | ok=false | ok=false | err:Disposed | err:WrongApp | err:WrongApp | ok=false |
| `untrack` | ok=1 | ok=1 | ok=1 | err:Disposed | err:WrongApp | err:WrongApp | - |
| `blocking` | ok | ok | ok | ok | ok | ok | - |
| `task::sleep` | ok | ok | ok | ok | ok | ok | - |
| `task::timeout` | ok | ok | ok | ok | ok | ok | - |
| `diagnostics::why` | ok=false | ok=true | ok=true | ok=false | err:WrongApp | err:WrongApp | - |
