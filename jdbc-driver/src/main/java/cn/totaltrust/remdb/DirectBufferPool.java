package cn.totaltrust.remdb;

import java.nio.ByteBuffer;
import java.util.concurrent.ConcurrentLinkedQueue;

/**
 * 直接内存缓冲池，用于零拷贝传输
 */
public class DirectBufferPool {
    private final ConcurrentLinkedQueue<ByteBuffer> pool;
    private final int bufferSize;
    private final int maxPoolSize;
    private int currentSize;

    /**
     * 创建直接内存缓冲池
     * @param poolSize 池大小
     * @param bufferSize 每个缓冲区大小
     */
    public DirectBufferPool(int poolSize, int bufferSize) {
        this.pool = new ConcurrentLinkedQueue<>();
        this.bufferSize = bufferSize;
        this.maxPoolSize = poolSize;
        this.currentSize = 0;
        
        // 预分配缓冲区
        for (int i = 0; i < poolSize; i++) {
            pool.offer(ByteBuffer.allocateDirect(bufferSize));
            currentSize++;
        }
    }

    /**
     * 获取直接内存缓冲区
     * @return 直接内存缓冲区
     */
    public ByteBuffer acquireBuffer() {
        // 尝试从池中获取
        ByteBuffer buffer = pool.poll();
        if (buffer != null) {
            buffer.clear();
            return buffer;
        }
        
        // 池中没有可用缓冲区，创建新的（如果未超过最大限制）
        if (currentSize < maxPoolSize) {
            currentSize++;
            return ByteBuffer.allocateDirect(bufferSize);
        }
        
        // 超过最大限制，创建临时缓冲区（会被GC回收）
        return ByteBuffer.allocateDirect(bufferSize);
    }

    /**
     * 释放缓冲区回池
     * @param buffer 要释放的缓冲区
     */
    public void releaseBuffer(ByteBuffer buffer) {
        if (buffer == null) {
            return;
        }
        
        // 只回收直接内存缓冲区
        if (buffer.isDirect()) {
            // 清空缓冲区
            buffer.clear();
            // 放回池中
            pool.offer(buffer);
        }
        // 非直接缓冲区会被GC自动回收
    }

    /**
     * 获取池大小
     * @return 池大小
     */
    public int getPoolSize() {
        return currentSize;
    }

    /**
     * 获取缓冲区大小
     * @return 缓冲区大小
     */
    public int getBufferSize() {
        return bufferSize;
    }

    /**
     * 关闭池，释放所有直接内存
     */
    public void close() {
        ByteBuffer buffer;
        while ((buffer = pool.poll()) != null) {
            // 释放直接内存
            buffer.clear();
            // 注意：Java中直接内存的释放是通过ByteBuffer的cleaner实现的
            // 调用clean()方法可以立即释放，否则会等待GC
            sun.misc.Cleaner cleaner = ((sun.nio.ch.DirectBuffer) buffer).cleaner();
            if (cleaner != null) {
                cleaner.clean();
            }
            currentSize--;
        }
    }
}
