CREATE TABLE job_queues (
    id SERIAL PRIMARY KEY,
    -- unique ID for the job
    job_type VARCHAR(255) NOT NULL,
    -- type of job to run
    job_data JSONB,
    -- data for the job (JSON or binary)
    job_schedule_id INTEGER,
    -- ID of the job schedule that created this job
    priority INTEGER NOT NULL,
    -- priority of the job (lower values are processed first)
    status VARCHAR(255) NOT NULL,
    -- status of the job (pending, in_progress, completed, failed)
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    -- time when the job was added to the queue
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(), 
    -- time when the job was last updated
    deleted_at TIMESTAMP, 
    -- time when the job was deleted
    FOREIGN KEY (job_schedule_id) REFERENCES job_schedules(id)
);