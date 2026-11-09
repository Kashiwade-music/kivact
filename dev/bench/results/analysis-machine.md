# KPI validation (2026-11-09T12:52Z, MINGW64_NT-10.0-22631, commit 757b73e)

分析機での参考値（`run.sh` v1.1、B0 の扱いではない）。この実行の間、OS が window の表示を約 32 fps に間引いた（全構成で interval が約 33 ms）。frame の work 時間は有効だが、fps と interval は参考にしない。

release build of kivact-example-counter (cold for its own target dir): 93 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=396.6 samples=[ 625.2 405.5 396.6 392.5 396.5 ]
- K4 first_frame_raster renderer=default median_ms=49.2 samples=[ 54.9 49.2 48.7 48.9 49.3 ]
- K1 main_to_first_present renderer=cpu median_ms=386.6 samples=[ 392.1 386.6 382.0 381.8 394.0 ]
- K4 first_frame_raster renderer=cpu median_ms=48.9 samples=[ 49.0 49.2 48.4 48.5 48.9 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=675.5 samples=[ 879.7 692.1 659.9 672.9 675.5 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=177176 private_kib=136036 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=Intel(R) UHD Graphics 770 (Vulkan) init_ms=84.9 configure_ms=24.0]
- counter renderer=cpu ws_kib=36448 private_kib=18324 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=177892 private_kib=136500 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=Intel(R) UHD Graphics 770 (Vulkan) init_ms=87.8 configure_ms=24.8]
- basic-catalog renderer=default ws_kib=395972 private_kib=287516 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=Intel(R) UHD Graphics 770 (Vulkan) init_ms=84.3 configure_ms=24.4]
- rows3000-still renderer=default ws_kib=610520 private_kib=647188 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=Intel(R) UHD Graphics 770 (Vulkan) init_ms=86.6 configure_ms=27.3]
- basic-catalog renderer=cpu ws_kib=166848 private_kib=105740 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- rows3000-still renderer=cpu ws_kib=477556 private_kib=545484 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=38960 private_kib=22472 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=124616 private_kib=113648 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=28.55
- item-text-0 renderer=cpu ws_kib=38980 private_kib=22480 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=70728 private_kib=58896 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=10.58
- item-button-0 renderer=cpu ws_kib=39220 private_kib=22484 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=365960 private_kib=382276 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=108.91
- item-row-0 renderer=cpu ws_kib=39248 private_kib=22496 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=476756 private_kib=544568 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=145.84

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=29185
    [box] H site   30632.9 B/item   104162 allocs 
    [box] H site    3501.9 B/item       88 allocs CallWindowProcW
    [box] H site       0.2 B/item        5 allocs KiUserCallbackDispatcher
- heap mode=text live_bytes_per_item=11534
    [text] H site   12992.2 B/item    66144 allocs 
    [text] H site    3508.8 B/item      363 allocs CallWindowProcW
    [text] H site       0.2 B/item        5 allocs KiUserCallbackDispatcher
- heap mode=button live_bytes_per_item=114644
    [button] H site  116110.5 B/item   876783 allocs 
    [button] H site    3507.3 B/item      119 allocs CallWindowProcW
    [button] H site       0.2 B/item        5 allocs KiUserCallbackDispatcher
- heap mode=row live_bytes_per_item=166339
    [row] H site  167461.9 B/item  1022582 allocs 
    [row] H site    3842.9 B/item      353 allocs CallWindowProcW
    [row] H site       0.2 B/item        5 allocs KiUserCallbackDispatcher

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default frames=175 fps=32.0 cpu_pct_of_core=0.00 ws_kib=196164 work_ms p50=0.7668 p95=2.098 p99=2.7287 scene_p95=0.069 raster_p95=1.9864 interval_p99=33.375
- rows=0 renderer=cpu frames=174 fps=31.9 cpu_pct_of_core=4.88 ws_kib=40644 work_ms p50=2.2035 p95=5.1983 p99=5.4159 scene_p95=0.061 raster_p95=4.9958 interval_p99=35.5738
- rows=3000 renderer=default frames=175 fps=32.1 cpu_pct_of_core=4.01 ws_kib=620140 work_ms p50=8.7627 p95=9.8093 p99=10.7857 scene_p95=8.8101 raster_p95=1.1692 interval_p99=33.0977
- rows=3000 renderer=cpu frames=174 fps=31.9 cpu_pct_of_core=6.31 ws_kib=477332 work_ms p50=8.7667 p95=9.8807 p99=10.686 scene_p95=8.2005 raster_p95=1.6309 interval_p99=34.2436

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=313.512 p50=313.512 p95=313.512 p99=313.512
- L counter_click n=200 mean=1.491 p50=1.463 p95=1.615 p99=2.069
- L mount_n100 n=1 mean=326.011 p50=326.011 p95=326.011 p99=326.011
- L unrelated_update_n100 n=50 mean=2.702 p50=2.652 p95=2.818 p99=4.679
- L row_update_n100 n=50 mean=12.016 p50=10.070 p95=34.594 p99=35.908
- L pointer_move_n100 n=120 mean=0.906 p50=0.302 p95=3.023 p99=3.529
- L wheel_n100 n=40 mean=0.257 p50=0.253 p95=0.284 p99=0.312
- S stats_n100 Stats { batches: 101, a11y_constructed: 2270, a11y_published: 1573, ops: 2853, last_batch_ops: 1, creates: 1223, set_props: 5609, structural: 1222, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 16524600, live_reactive: 5750, live_owners: 101, live_objects: 409, live_constants: 612, arena_slots: 6872, tree_nodes: 1223, timers: 0, frame_loops: 102, animations: 0 }
- L mount_n1000 n=1 mean=538.230 p50=538.230 p95=538.230 p99=538.230
- L unrelated_update_n1000 n=50 mean=13.067 p50=12.782 p95=14.786 p99=18.579
- L row_update_n1000 n=50 mean=25.252 p50=23.632 p95=46.813 p99=48.403
- L pointer_move_n1000 n=120 mean=6.786 p50=3.640 p95=17.961 p99=18.926
- L wheel_n1000 n=40 mean=3.085 p50=3.049 p95=3.388 p99=3.665
- S stats_n1000 Stats { batches: 101, a11y_constructed: 13070, a11y_published: 12373, ops: 27153, last_batch_ops: 1, creates: 12023, set_props: 54209, structural: 12022, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 85852900, live_reactive: 56150, live_owners: 1001, live_objects: 4009, live_constants: 6012, arena_slots: 67172, tree_nodes: 12023, timers: 0, frame_loops: 1002, animations: 0 }
- L mount_n3000 n=1 mean=993.660 p50=993.660 p95=993.660 p99=993.660
- L unrelated_update_n3000 n=50 mean=48.474 p50=47.756 p95=55.980 p99=59.673
- L row_update_n3000 n=50 mean=66.301 p50=64.772 p95=86.906 p99=90.782
- L pointer_move_n3000 n=120 mean=25.396 p50=14.235 p95=64.824 p99=68.783
- L wheel_n3000 n=40 mean=12.870 p50=12.605 p95=15.034 p99=15.066
- S stats_n3000 Stats { batches: 101, a11y_constructed: 37070, a11y_published: 36373, ops: 81153, last_batch_ops: 1, creates: 36023, set_props: 162209, structural: 36022, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 284661600, live_reactive: 168150, live_owners: 3001, live_objects: 12009, live_constants: 18012, arena_slots: 201172, tree_nodes: 36023, timers: 0, frame_loops: 3002, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 24604160
- K6 kv_startup default 24600576
- K6 kv_startup no-default-features 24600576
- K6 basic-catalog default 54252032
