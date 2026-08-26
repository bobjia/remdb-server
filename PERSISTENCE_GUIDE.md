# remdb-server Data Persistence Guide

## ⚠️ Critical: Preventing Data Loss

### The Problem with Incremental-Only Snapshots

If you use `snapshot_type = "incremental"` exclusively without ever creating a full snapshot:

1. **Incremental snapshots cannot be used for recovery** - they require a full snapshot as a base
2. **All your incremental snapshots are useless** if no full snapshot exists
3. **Risk of complete data loss** if WAL file is corrupted and no full snapshot exists

### Current Status Check

```bash
# Check for full snapshots (should have at least one!)
ls db/snapshot/full_*.remd

# Check WAL file
ls -lh wal/remdb.wal

# Count incremental snapshots
ls db/snapshot/incremental_*.remd | wc -l
```

## Recommended Snapshot Strategy

### Option 1: Full Snapshots Only (Simplest & Safest)

```toml
snapshot_type = "full"
snapshot_interval = 3600  # Every hour
max_incremental_snapshots = 10
```

**Pros:**
- ✅ Simple recovery: just load the latest full snapshot
- ✅ No dependency chain issues
- ✅ Each snapshot is self-contained

**Cons:**
- ❌ Larger disk usage
- ❌ Slower snapshot creation

### Option 2: Alternating Full + Incremental (Balanced)

Manually alternate or use a cron job:

```bash
# Weekly full snapshot (Sunday midnight)
0 0 * * 0 /path/to/remdbcli -c "snapshot full"

# Hourly incremental snapshots (Mon-Sat)
0 * * * 1-6 /path/to/remdbcli -c "snapshot incremental"
```

Configuration:
```toml
# Set to incremental, but ensure weekly full snapshots via cron
snapshot_type = "incremental"
snapshot_interval = 3600
max_incremental_snapshots = 168  # Keep 1 week of hourly incrementals
```

### Option 3: Manual Control (Current Setup)

Keep automatic incremental snapshots but manually trigger full snapshots:

```toml
snapshot_type = "incremental"
snapshot_interval = 60
max_incremental_snapshots = 10
```

Then run periodically:
```bash
./create-full-snapshot.sh
```

## Recovery Process

The server recovers data in this priority order:

1. **Full Image File** (if specified via `--full-image` flag)
2. **WAL Directory** → Loads latest snapshot/checkpoint + replays WAL
3. **Snapshot Directory** → Fallback if WAL doesn't exist

### Recovery Requirements

For successful recovery from snapshots, you need:
- ✅ At least ONE full snapshot (`full_*.remd`)
- ✅ OR checkpoints (`checkpoint_*`)
- ❌ Incremental snapshots alone are NOT sufficient

## Immediate Actions Required

### 1. Create a Full Snapshot NOW

```bash
# Method 1: Using the helper script
./create-full-snapshot.sh

# Method 2: Using remdbcli interactively
cargo run --bin remdbcli
# Then type: snapshot full

# Method 3: Direct SQL command (if JDBC connected)
# Not available yet - use CLI
```

### 2. Verify Snapshot Creation

```bash
ls -lh db/snapshot/
# Should see: full_<timestamp>.remd
```

### 3. Test Recovery (Optional but Recommended)

```bash
# Stop the server
# Backup current WAL
cp wal/remdb.wal wal/remdb.wal.backup

# Remove WAL to test snapshot-only recovery
rm wal/remdb.wal

# Restart server - should recover from snapshot
cargo run --bin remdb-server

# Verify data integrity
# ... check your tables ...

# Restore WAL if needed
mv wal/remdb.wal.backup wal/remdb.wal
```

## Monitoring & Maintenance

### Check Snapshot Health

```bash
#!/bin/bash
# save as check-snapshots.sh

echo "=== Snapshot Status ==="
echo "Full snapshots:"
ls -lh db/snapshot/full_*.remd 2>/dev/null || echo "  NONE! ⚠️"

echo ""
echo "Incremental snapshots:"
ls -lh db/snapshot/incremental_*.remd 2>/dev/null | tail -5 || echo "  None"

echo ""
echo "WAL file:"
ls -lh wal/remdb.wal 2>/dev/null || echo "  NONE! ⚠️"

echo ""
echo "Total snapshot size:"
du -sh db/snapshot/ 2>/dev/null || echo "  N/A"
```

### Automated Full Snapshot Reminder

Add to crontab:
```cron
# Remind to create full snapshot every Sunday
0 9 * * 0 echo "⚠️ REMINDER: Create a full snapshot this week!" | mail -s "remdb Full Snapshot Reminder" admin@example.com
```

## Disk Space Management

### Estimate Snapshot Sizes

- **Full snapshot**: ~Size of all data in memory
- **Incremental snapshot**: ~Size of changes since last snapshot (typically much smaller)

### Cleanup Old Snapshots

The system automatically cleans up old incremental snapshots when exceeding `max_incremental_snapshots`.

To manually clean:
```bash
# Remove old full snapshots (keep last 3)
ls -t db/snapshot/full_*.remd | tail -n +4 | xargs rm -f

# Remove old incremental snapshots (keep last 10)
ls -t db/snapshot/incremental_*.remd | tail -n +11 | xargs rm -f
```

## Emergency Recovery Scenarios

### Scenario 1: WAL Corrupted, Have Full Snapshot

```bash
# Server will automatically:
# 1. Load latest full snapshot
# 2. Skip WAL replay (corrupted)
# Result: Data up to last full snapshot ✅
```

### Scenario 2: No Full Snapshot, Only Incrementals

```bash
# Server will:
# 1. Look for full snapshot → NOT FOUND
# 2. Skip all incremental snapshots
# 3. Try WAL → If WAL exists, recover from it
# Result: Data loss if WAL also missing ❌
```

### Scenario 3: Everything Lost

```bash
# No snapshots, no WAL
# Result: Complete data loss, start fresh ❌
```

## Best Practices Summary

1. ✅ **Always maintain at least one recent full snapshot**
2. ✅ **Monitor snapshot directory regularly**
3. ✅ **Test recovery procedure periodically**
4. ✅ **Backup snapshots to remote storage**
5. ✅ **Keep WAL file on reliable storage (SSD preferred)**
6. ❌ **Never rely solely on incremental snapshots**
7. ❌ **Don't ignore snapshot warnings in logs**

## Quick Reference Commands

```bash
# Create full snapshot
./create-full-snapshot.sh

# Check snapshot status
ls -lh db/snapshot/

# View server logs for snapshot activity
tail -f logs/remdb-server-*.log | grep -i snapshot

# Monitor WAL size
watch -n 5 'ls -lh wal/remdb.wal'
```
