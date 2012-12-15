(ns chapter-tracker.view (:gen-class)
  ;(:use seesaw.core)
  (:use chapter-tracker.view.main-frame)
)

(import
  '(javax.swing UIManager)
  '(javax.swing.plaf FontUIResource)
)

(when-not (resolve 'main-frame)
  (def main-frame nil)
)
(defn show-frame []
  (if main-frame
    (try
      (.hide main-frame)
      (.dispose main-frame)
      (catch Exception e)))

  (doseq [ui-key (-> (UIManager/getDefaults) .keys enumeration-seq)]
    (let [ui-value (UIManager/get ui-key)]
      (when (instance? FontUIResource ui-value)
        (UIManager/put ui-key (FontUIResource. "Ariel" FontUIResource/BOLD 14))
      )
    )
  )
  (def main-frame (create-main-frame))
  (.show main-frame)
)
