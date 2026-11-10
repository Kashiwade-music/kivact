# KPI B0 (2026-11-09T13:30Z, MINGW64_NT-10.0-26200, commit 79bbd409)

release build of kivact-example-counter (cold for its own target dir): 1 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=461.8 samples=[ 3640.8 461.0 464.1 459.8 461.8 ]
- K4 first_frame_raster renderer=default median_ms=30.7 samples=[ 35.8 30.0 30.0 30.9 30.7 ]
- K1 main_to_first_present renderer=cpu median_ms=436.1 samples=[ 448.7 436.1 423.1 437.8 433.1 ]
- K4 first_frame_raster renderer=cpu median_ms=29.7 samples=[ 29.9 29.7 29.9 29.6 29.6 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=654.0 samples=[ 986.0 650.8 650.4 658.8 654.0 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=136068 private_kib=132380 idle_cpu_ms_per_s=1.500 idle_frames=0 cpu_ticks_100ns=156250 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=786.8 configure_ms=15.7]
- counter renderer=cpu ws_kib=37372 private_kib=14084 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- counter renderer=gpu ws_kib=136444 private_kib=133792 idle_cpu_ms_per_s=2.999 idle_frames=0 cpu_ticks_100ns=312500 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=816.1 configure_ms=14.3]
- basic-catalog renderer=default ws_kib=257096 private_kib=241916 idle_cpu_ms_per_s=187.210 idle_frames=625 cpu_ticks_100ns=19531250 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=1012.2 configure_ms=15.2]
- rows3000-still renderer=default ws_kib=649944 private_kib=761892 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 [renderer=gpu adapter=NVIDIA GeForce RTX 3060 Ti (Vulkan) init_ms=1013.7 configure_ms=17.8]
- basic-catalog renderer=cpu ws_kib=162616 private_kib=96288 idle_cpu_ms_per_s=184.475 idle_frames=625 cpu_ticks_100ns=19218750 
- rows3000-still renderer=cpu ws_kib=554676 private_kib=618392 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=38048 private_kib=16280 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-box-3000 renderer=cpu ws_kib=135620 private_kib=119268 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=box kib_per_item=32.52
- item-text-0 renderer=cpu ws_kib=38012 private_kib=16272 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-text-3000 renderer=cpu ws_kib=81712 private_kib=65016 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=text kib_per_item=14.57
- item-button-0 renderer=cpu ws_kib=38064 private_kib=17092 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-button-3000 renderer=cpu ws_kib=409312 private_kib=420800 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=button kib_per_item=123.75
- item-row-0 renderer=cpu ws_kib=38044 private_kib=17100 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- item-row-3000 renderer=cpu ws_kib=554776 private_kib=618468 idle_cpu_ms_per_s=0.000 idle_frames=0 cpu_ticks_100ns=0 
- K2 per_item mode=row kib_per_item=172.24

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=32541
    [box] H site   30451.8 B/item   111641 allocs 
    [box] H site    5551.5 B/item    22633 allocs CallWindowProcW
    [box] H site       0.4 B/item        3 allocs DefSubclassProc
    [box] H site       0.2 B/item        5 allocs KiUserCallbackDispatcher
    [box] H site       0.2 B/item        1 allocs IsWindowEnabled
- heap mode=text live_bytes_per_item=15030
    [text] H site   12830.2 B/item    79602 allocs 
    [text] H site    5660.9 B/item    31902 allocs CallWindowProcW
    [text] H site       0.4 B/item        3 allocs DefSubclassProc
    [text] H site       0.2 B/item        1 allocs IsWindowEnabled
    [text] H site       0.2 B/item        5 allocs KiUserCallbackDispatcher
- heap mode=button live_bytes_per_item=128695
    [button] H site  116960.0 B/item   914218 allocs 
    [button] H site   15201.5 B/item   119507 allocs CallWindowProcW
    [button] H site       0.3 B/item        2 allocs DefSubclassProc
    [button] H site       0.2 B/item        5 allocs KiUserCallbackDispatcher
    [button] H site       0.2 B/item        1 allocs IsWindowEnabled
- heap mode=row live_bytes_per_item=191959
    [row] H site  168883.0 B/item  1075020 allocs 
    [row] H site   26550.6 B/item   165325 allocs CallWindowProcW
    [row] H site       0.3 B/item        2 allocs DefSubclassProc
    [row] H site       0.2 B/item        1 allocs IsWindowEnabled
    [row] H site       0.2 B/item        5 allocs KiUserCallbackDispatcher

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default frames=328 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=15.43 ws_kib=149324 work_ms p50=2.1282 p95=3.5387 p99=4.0195 scene_p95=0.0466 raster_p95=3.2219 interval_p99=18.5697
- rows=0 renderer=cpu frames=326 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=12.94 ws_kib=40704 work_ms p50=1.7218 p95=2.1117 p99=2.2061 scene_p95=0.0482 raster_p95=1.9776 interval_p99=17.3345
  WARNING rows=3000 renderer=default: 45.2 fps for a 60.0 Hz display (the OS throttled the window; work times stay valid, fps and interval do not)
- rows=3000 renderer=default frames=246 fps=45.2 frame_period_ns=16666666 cpu_pct_of_core=99.37 ws_kib=577332 work_ms p50=19.8455 p95=21.9907 p99=24.7097 scene_p95=19.1643 raster_p95=3.5192 interval_p99=27.1347
  WARNING rows=3000 renderer=cpu: 47.2 fps for a 60.0 Hz display (the OS throttled the window; work times stay valid, fps and interval do not)
- rows=3000 renderer=cpu frames=256 fps=47.2 frame_period_ns=16666666 cpu_pct_of_core=99.57 ws_kib=475700 work_ms p50=19.2876 p95=20.9544 p99=22.4047 scene_p95=18.957 raster_p95=2.3381 interval_p99=24.2412

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=373.924 p50=373.924 p95=373.924 p99=373.924
- L counter_click n=200 mean=2.288 p50=2.165 p95=2.786 p99=3.140
- L mount_n100 n=1 mean=408.438 p50=408.438 p95=408.438 p99=408.438
- L unrelated_update_n100 n=50 mean=4.071 p50=3.994 p95=4.643 p99=6.749
- L row_update_n100 n=50 mean=14.358 p50=12.929 p95=33.973 p99=34.151
- L pointer_move_n100 n=120 mean=1.389 p50=0.536 p95=4.483 p99=4.900
- L wheel_n100 n=40 mean=0.444 p50=0.436 p95=0.558 p99=0.584
- S stats_n100 Stats { batches: 101, a11y_constructed: 2270, a11y_published: 1573, ops: 2853, last_batch_ops: 1, creates: 1223, set_props: 5609, structural: 1222, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 25469700, live_reactive: 5750, live_owners: 101, live_objects: 409, live_constants: 612, arena_slots: 6872, tree_nodes: 1223, timers: 0, frame_loops: 102, animations: 0 }
- L mount_n1000 n=1 mean=741.572 p50=741.572 p95=741.572 p99=741.572
- L unrelated_update_n1000 n=50 mean=22.598 p50=22.316 p95=23.901 p99=28.849
- L row_update_n1000 n=50 mean=36.951 p50=35.632 p95=57.447 p99=58.375
- L pointer_move_n1000 n=120 mean=13.174 p50=8.012 p95=31.242 p99=31.839
- L wheel_n1000 n=40 mean=6.968 p50=7.006 p95=7.803 p99=7.915
- S stats_n1000 Stats { batches: 101, a11y_constructed: 13070, a11y_published: 12373, ops: 27153, last_batch_ops: 1, creates: 12023, set_props: 54209, structural: 12022, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 168299800, live_reactive: 56150, live_owners: 1001, live_objects: 4009, live_constants: 6012, arena_slots: 67172, tree_nodes: 12023, timers: 0, frame_loops: 1002, animations: 0 }
- L mount_n3000 n=1 mean=1627.522 p50=1627.522 p95=1627.522 p99=1627.522
- L unrelated_update_n3000 n=50 mean=75.678 p50=75.429 p95=76.661 p99=84.588
- L row_update_n3000 n=50 mean=101.739 p50=100.324 p95=122.021 p99=122.987
- L pointer_move_n3000 n=120 mean=44.237 p50=25.108 p95=105.989 p99=106.261
- L wheel_n3000 n=40 mean=24.436 p50=23.887 p95=28.291 p99=29.487
- S stats_n3000 Stats { batches: 101, a11y_constructed: 37070, a11y_published: 36373, ops: 81153, last_batch_ops: 1, creates: 36023, set_props: 162209, structural: 36022, host_touched_last: 1, turns: 222, stale_events: 0, core_ns: 478659300, live_reactive: 168150, live_owners: 3001, live_objects: 12009, live_constants: 18012, arena_slots: 201172, tree_nodes: 36023, timers: 0, frame_loops: 3002, animations: 0 }

## K6 release binary size (bytes)
- K6 counter default 24594432
- K6 kv_startup default 24590848
- K6 kv_startup no-default-features 24590848
- K6 basic-catalog default 54251520

- note: V1 で同じ機械・同じ commit 79bbd409 で測った B0.md を B0-here として使う（機械・commit・条件が同じ）。
