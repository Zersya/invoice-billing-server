CREATE TABLE job_schedules (
    id SERIAL PRIMARY KEY,
    -- unique ID for the job
    job_type VARCHAR(255) NOT NULL,
    -- type of job to run (send_invoice, send_reminder)
    job_data JSONB,
    -- data for the job (JSON or binary)
    run_at TIMESTAMP NOT NULL,
    -- time when the job should be run
    repeat_interval INTEGER,
    -- interval at which the job should be repeated (in seconds)
    repeat_count INTEGER,
    -- number of times the job should be repeated (NULL for unlimited)
    dependencies TEXT,
    -- IDs of other jobs that must be completed first
    status VARCHAR(255) NOT NULL,
    -- status of the job (scheduled, pending, in_progress, completed, failed)
    retry_count INTEGER,
    -- number of times the job has been retried (NULL for unlimited)
    retry_interval INTEGER,
    -- interval at which the job should be retried (in seconds)
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    -- time when the job was added to the schedule
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    -- time when the job was last updated
    deleted_at TIMESTAMP 
    -- time when the job was deleted
);