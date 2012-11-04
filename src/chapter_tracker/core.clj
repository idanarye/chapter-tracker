(ns chapter-tracker.core (:gen-class)
  (:use chapter-tracker.view)
  (:use chapter-tracker.model)
  (:use chapter-tracker.schema)
)

(defn print-clj-trace [exception]
  (println (.toString exception))
  (doseq [trace-element (.getStackTrace exception) :when (.. trace-element getClassName (startsWith "chapter"))]
    (println \tab (.toString trace-element))
  )
)

(defn -main[]
  (try
    (create-tables)
    (catch Exception e))
  (try
    (show-frame)
    (catch Exception e
      (print-clj-trace e)
    )
  )
)
